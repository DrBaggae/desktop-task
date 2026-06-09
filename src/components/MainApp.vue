<template>
  <div class="main-app">
    <h1>Task Manager</h1>

    <div class="toolbar">
      <input v-model="search" placeholder="Search tasks..." />
      <select v-model="filterStatus">
        <option value="">All Status</option>
        <option value="pending">Pending</option>
        <option value="in_progress">In Progress</option>
        <option value="completed">Completed</option>
      </select>
      <select v-model="filterPriority">
        <option value="">All Priority</option>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
      </select>
      <button @click="showForm = true">+ Add Task</button>
    </div>

    <div v-if="showForm" class="form">
      <input v-model="newTask.title" placeholder="Task title" />
      <textarea v-model="newTask.description" placeholder="Description"></textarea>
      <input v-model="newTask.due_date" type="date" />
      <select v-model="newTask.priority">
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
      </select>
      <select v-model="newTask.status">
        <option value="pending">Pending</option>
        <option value="in_progress">In Progress</option>
        <option value="completed">Completed</option>
      </select>
      <div class="form-buttons">
        <button @click="addTask">Save</button>
        <button @click="showForm = false">Cancel</button>
      </div>
    </div>

    <table>
      <thead>
        <tr>
          <th>Title</th>
          <th>Status</th>
          <th>Priority</th>
          <th>Due Date</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="task in filteredTasks" :key="task.id">
          <td>{{ task.title }}</td>
          <td>{{ task.status }}</td>
          <td>{{ task.priority }}</td>
          <td>{{ task.due_date }}</td>
          <td>
            <button @click="completeTask(task.id)">Complete</button>
            <button @click="deleteTask(task.id)">Delete</button>
          </td>
        </tr>
        <tr v-if="filteredTasks.length === 0">
          <td colspan="5">No tasks found.</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/core'

export default {
  data() {
    return {
      tasks: [],
      search: '',
      filterStatus: '',
      filterPriority: '',
      showForm: false,
      newTask: {
        title: '',
        description: '',
        due_date: '',
        priority: 'medium',
        status: 'pending',
        completed: false,
      }
    }
  },
  computed: {
    filteredTasks() {
      return this.tasks.filter(t => {
        const matchSearch = t.title.toLowerCase().includes(this.search.toLowerCase())
        const matchStatus = this.filterStatus ? t.status === this.filterStatus : true
        const matchPriority = this.filterPriority ? t.priority === this.filterPriority : true
        return matchSearch && matchStatus && matchPriority
      })
    }
  },
  async mounted() {
    await this.loadTasks()
  },
  methods: {
    async loadTasks() {
      this.tasks = await invoke('get_tasks')
    },
    async addTask() {
      const task = {
        ...this.newTask,
        id: Date.now().toString(),
      }
      await invoke('add_task', { task })
      await this.loadTasks()
      this.showForm = false
      this.newTask = { title: '', description: '', due_date: '', priority: 'medium', status: 'pending', completed: false }
    },
    async completeTask(id) {
      await invoke('complete_task', { id })
      await this.loadTasks()
    },
    async deleteTask(id) {
      await invoke('delete_task', { id })
      await this.loadTasks()
    },
  }
}
</script>

<style scoped>
.main-app {
  padding: 20px;
  font-family: sans-serif;
}

h1 {
  margin-bottom: 16px;
}

.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.toolbar input, .toolbar select {
  padding: 6px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.toolbar button {
  padding: 6px 12px;
  background: #2d7a9a;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
  background: #f5f5f5;
  padding: 12px;
  border-radius: 8px;
}

.form input, .form select, .form textarea {
  padding: 6px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.form-buttons {
  display: flex;
  gap: 8px;
}

.form-buttons button {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  padding: 10px;
  border: 1px solid #ddd;
  text-align: left;
}

th {
  background: #f0f0f0;
}

td button {
  margin-right: 4px;
  padding: 4px 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}
</style>